package cz.mgn.irro

import android.os.Bundle
import androidx.fragment.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Switch
import cz.mgn.irro.api.IrroApiService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch


class SwitchesFragment : Fragment() {

    private var viewModelJob = Job()
    private val coroutineScope = CoroutineScope(viewModelJob + Dispatchers.IO)
    private lateinit var api: IrroApiService

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        // Inflate the layout for this fragment
        val fragmentView = inflater.inflate(R.layout.fragment_switches, container, false)

        api = (activity as RemoteActivity).api

        val onboardLedSwitch = fragmentView.findViewById<Switch>(R.id.onboard_switch)
        onboardLedSwitch?.setOnCheckedChangeListener { compoundButton, checked ->
            coroutineScope.launch {
                api.setLed(0, checked).await()
            }
        }

        return fragmentView
    }

    override fun onDestroy() {
        super.onDestroy()
        viewModelJob.cancel()
    }

    private fun loadSwitches() {
        coroutineScope.launch {
            api.g
        }
    }

    companion object {
        /**
         * Use this factory method to create a new instance of
         * this fragment using the provided parameters.
         *
         * @return A new instance of fragment DiodsFragment.
         */
        @JvmStatic
        fun newInstance() =
            SwitchesFragment().apply {
                arguments = Bundle()
            }
    }
}
