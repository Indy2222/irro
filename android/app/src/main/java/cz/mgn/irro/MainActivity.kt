package cz.mgn.irro

import android.app.AlertDialog
import android.content.DialogInterface
import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.EditText
import kotlinx.coroutines.*
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.SocketTimeoutException
import android.text.InputType
import android.view.View
import android.widget.ProgressBar
import com.google.android.material.floatingactionbutton.FloatingActionButton

class MainActivity : AppCompatActivity() {

    private val broadcastPort = 34254
    private val broadcastTimeoutMs = 60_000
    private val findIrroJob = Job()
    private val uiScope = CoroutineScope(Dispatchers.Main + findIrroJob)
    private lateinit var progressBar: ProgressBar

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        progressBar = findViewById(R.id.irro_discovery_progress_bar)

        val cancelButton = findViewById<FloatingActionButton>(R.id.cancel_discovery_button)

        cancelButton.setOnClickListener { view ->
            setHostManually()
        }

        findIrro()
    }

    override fun onDestroy() {
        super.onDestroy()
        findIrroJob.cancel()
    }

    private fun findIrro() {
        uiScope.launch {
            progressBar.visibility = View.VISIBLE
            val irroHost = interceptIrroBroadcast()
            progressBar.visibility = View.GONE

            if (irroHost == null) {
                displayNotFound()
            } else {
                moveToControl(irroHost)
            }
        }

    }

    private fun moveToControl(irroHost: String) {
        val intent = Intent(this, RemoteActivity::class.java)
        intent.putExtra("irroHost", irroHost)
        startActivity(intent)
        finish()
    }

    private fun displayNotFound() {
        val builder = AlertDialog.Builder(this)
        builder.setMessage(R.string.irro_not_found_text).setTitle(R.string.irro_not_found_title)

        builder.apply {
            setPositiveButton(R.string.retry,
                DialogInterface.OnClickListener { dialog, id ->
                    findIrro()
                })
            setNeutralButton(R.string.set_manually,
                DialogInterface.OnClickListener { dialog, id ->
                    setHostManually()
                })
            setNegativeButton(R.string.cancel,
                DialogInterface.OnClickListener { dialog, id ->
                    finishAffinity()
                })
        }

        builder.show()
    }

    private fun setHostManually() {
        findIrroJob.cancel()
        progressBar.visibility = View.GONE

        val hostInput = EditText(this)
        hostInput.setInputType(InputType.TYPE_CLASS_NUMBER)

        val dialog = AlertDialog.Builder(this)
            .setTitle(R.string.input_host)
            .setView(hostInput)
            .setPositiveButton(R.string.ok, null)
            .create()

        dialog.setOnShowListener(DialogInterface.OnShowListener { _ ->
            val positiveButton = dialog.getButton(AlertDialog.BUTTON_POSITIVE)
            positiveButton.setOnClickListener(View.OnClickListener {
                if (hostInput.text.isEmpty()) {
                    hostInput.error = "Host can't be empty."
                } else {
                    dialog.dismiss()
                    val irroHost = hostInput.text.toString()
                    moveToControl(irroHost)
                }
            })
        })

        dialog.show()
    }

    private suspend fun interceptIrroBroadcast(): String? {
        return withContext(Dispatchers.IO) {
            val socket = DatagramSocket(broadcastPort)
            socket.soTimeout = broadcastTimeoutMs

            val buffer = ByteArray(64)
            val packet = DatagramPacket(buffer, 50)
            var host: String? = null

            try {
                socket.receive(packet)
                host = packet.getAddress().getHostAddress()
            } catch (e: SocketTimeoutException) {
                // Return null in this case.
            } finally {
                socket.close()
            }

            host
        }
    }
}
