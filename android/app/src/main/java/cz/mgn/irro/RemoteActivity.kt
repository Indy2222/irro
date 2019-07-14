package cz.mgn.irro

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.TextView

class RemoteActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_remote)

        val irroHostTextView = findViewById<TextView>(R.id.irro_host_text_view)
        irroHostTextView.text = intent.getStringExtra("irroHost")
    }
}
