package cz.mgn.irro

import android.app.AlertDialog
import android.content.DialogInterface
import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import kotlinx.coroutines.*
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.SocketTimeoutException

class MainActivity : AppCompatActivity() {

    private val broadcastPort = 34254
    private val broadcastTimeoutMs = 60_000
    private val findIrroJob = Job()
    private val uiScope = CoroutineScope(Dispatchers.Main + findIrroJob)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        findIrro()
    }

    override fun onDestroy() {
        super.onDestroy()
        findIrroJob.cancel()
    }

    private fun findIrro() {
        val context = this
        uiScope.launch {
            val irroHost = interceptIrroBroadcast()

            if (irroHost == null) {
                displayNotFound()
            } else {
                val intent = Intent(context, RemoteActivity::class.java)
                intent.putExtra("irroHost", irroHost)
                startActivity(intent)
                finish()
            }
        }

    }

    private fun displayNotFound() {
        val builder = AlertDialog.Builder(this)
        builder.setMessage(R.string.irro_not_found_text).setTitle(R.string.irro_not_found_title)

        builder.apply {
            setPositiveButton(R.string.retry,
                DialogInterface.OnClickListener { dialog, id ->
                    findIrro()
                })
            setNegativeButton(R.string.cancel,
                DialogInterface.OnClickListener { dialog, id ->
                    finishAffinity()
                })
        }


        builder.create()
        builder.show()
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
