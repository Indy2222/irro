package cz.mgn.irro.api

import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory
import retrofit2.http.PUT
import retrofit2.http.GET
import com.jakewharton.retrofit2.adapter.kotlin.coroutines.CoroutineCallAdapterFactory
import kotlinx.coroutines.Deferred
import retrofit2.http.Body
import retrofit2.http.Path

interface IrroApiService {
    @GET("low/led")
    fun getLed(): Deferred<List<Boolean>>

    @PUT("low/led/{ledId}")
    fun setLed(
        @Path("ledId") ledId: Int,
        @Body value: Boolean
    ): Deferred<Unit>
}

fun initApiService(irroHost: String): IrroApiService {
    val baseUrl = "http://$irroHost:8080/"

    val moshi = Moshi.Builder()
        .add(KotlinJsonAdapterFactory())
        .build()

    val retrofit = Retrofit.Builder()
        .addConverterFactory(MoshiConverterFactory.create(moshi))
        .addCallAdapterFactory(CoroutineCallAdapterFactory())
        .baseUrl(baseUrl)
        .build()

    return retrofit.create<IrroApiService>(IrroApiService::class.java)
}
