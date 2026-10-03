package app.framegrab.mobile

import android.app.Activity
import android.content.ContentValues
import android.os.Environment
import android.provider.MediaStore
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.chaquo.python.Python
import com.chaquo.python.android.AndroidPlatform
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

@InvokeArg
class DownloadArgs {
    lateinit var id: String
    lateinit var url: String
}

@InvokeArg
class CancelArgs {
    lateinit var id: String
}

interface ProgressSink {
    fun onProgress(percent: Double)
    fun isCancelled(): Boolean
}

@TauriPlugin
class MediaPlugin(private val activity: Activity) : Plugin(activity) {
    private val executor = Executors.newSingleThreadExecutor()
    private val lock = Any()
    private var activeId: String? = null
    private var cancelled = AtomicBoolean(false)

    private fun event(id: String, status: String, message: String, percent: Double? = null) {
        val payload = JSObject()
        payload.put("id", id)
        payload.put("status", status)
        payload.put("message", message)
        payload.put("percent", percent)
        trigger("download-event", payload)
    }

    @Command
    fun startDownload(invoke: Invoke) {
        val args = try {
            invoke.parseArgs(DownloadArgs::class.java)
        } catch (error: Exception) {
            invoke.reject("Invalid download request.")
            return
        }
        if (!args.id.matches(Regex("[a-fA-F0-9-]{36}")) || !args.url.startsWith("https://")) {
            invoke.reject("Invalid download request.")
            return
        }
        val flag = AtomicBoolean(false)
        synchronized(lock) {
            if (activeId != null) {
                invoke.reject("Wait for the current download to finish.")
                return
            }
            activeId = args.id
            cancelled = flag
        }
        val response = JSObject()
        response.put("id", args.id)
        invoke.resolve(response)

        executor.execute {
            val work = File(activity.cacheDir, "downloads/${args.id}")
            var finalStatus = "failed"
            var finalMessage = "No compatible MP4 was available, or the link could not be downloaded."
            var finalPercent: Double? = null
            try {
                if (!Python.isStarted()) {
                    Python.start(AndroidPlatform(activity.applicationContext))
                }
                event(args.id, "downloading", "Finding a compatible MP4…")
                val sink = object : ProgressSink {
                    override fun onProgress(percent: Double) {
                        event(args.id, "downloading", "Downloading…", percent)
                    }
                    override fun isCancelled() = flag.get()
                }
                val path = Python.getInstance().getModule("framegrab_download")
                    .callAttr("download", args.url, work.absolutePath, sink).toString()
                if (!flag.get()) {
                    event(args.id, "processing", "Saving to Movies / Framegrab…")
                    saveToMovies(File(path), flag)
                }
                if (flag.get()) {
                    finalStatus = "cancelled"
                    finalMessage = "Download cancelled."
                } else {
                    finalStatus = "completed"
                    finalMessage = "Movies / Framegrab"
                    finalPercent = 100.0
                }
            } catch (error: Exception) {
                if (flag.get()) {
                    finalStatus = "cancelled"
                    finalMessage = "Download cancelled."
                }
            } finally {
                work.deleteRecursively()
                synchronized(lock) {
                    if (activeId == args.id) activeId = null
                }
            }
            event(args.id, finalStatus, finalMessage, finalPercent)
        }
    }

    @Command
    fun cancelDownload(invoke: Invoke) {
        val id = try {
            invoke.parseArgs(CancelArgs::class.java).id
        } catch (error: Exception) {
            invoke.reject("Invalid download identifier.")
            return
        }
        synchronized(lock) {
            if (activeId != id) {
                invoke.reject("This download is no longer active.")
                return
            }
            cancelled.set(true)
        }
        invoke.resolve(JSObject())
    }

    private fun saveToMovies(file: File, flag: AtomicBoolean) {
        if (!file.isFile || file.extension.lowercase() != "mp4") {
            throw IllegalArgumentException("No completed MP4 video was produced.")
        }
        val resolver = activity.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.Video.Media.DISPLAY_NAME, file.name)
            put(MediaStore.Video.Media.MIME_TYPE, "video/mp4")
            put(MediaStore.Video.Media.RELATIVE_PATH, "${Environment.DIRECTORY_MOVIES}/Framegrab")
            put(MediaStore.Video.Media.IS_PENDING, 1)
        }
        val uri = resolver.insert(MediaStore.Video.Media.EXTERNAL_CONTENT_URI, values)
            ?: throw IllegalStateException("Could not create video in Movies.")
        try {
            resolver.openOutputStream(uri)?.use { output ->
                file.inputStream().use { input ->
                    val buffer = ByteArray(64 * 1024)
                    while (true) {
                        if (flag.get()) throw IllegalStateException("Download cancelled.")
                        val count = input.read(buffer)
                        if (count < 0) break
                        output.write(buffer, 0, count)
                    }
                }
            } ?: throw IllegalStateException("Could not open the destination video.")
            if (flag.get()) throw IllegalStateException("Download cancelled.")
            values.clear()
            values.put(MediaStore.Video.Media.IS_PENDING, 0)
            resolver.update(uri, values, null, null)
        } catch (error: Exception) {
            resolver.delete(uri, null, null)
            throw error
        }
    }
}
