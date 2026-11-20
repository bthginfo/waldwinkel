package game.waldwinkel;

import android.app.NativeActivity;
import android.content.Intent;
import android.hardware.input.InputManager;
import android.net.Uri;
import android.os.Build;
import android.os.Handler;
import android.os.Looper;
import android.view.DisplayCutout;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.concurrent.atomic.AtomicBoolean;

/** Android window, device and document integration for the Rust game. */
public final class GameActivity extends NativeActivity implements InputManager.InputDeviceListener {
    private static final int SAVE_DOCUMENT = 510;
    private static final int READ_DOCUMENT = 511;
    private static final int MAX_SAVE_BYTES = 4 * 1024 * 1024;
    private volatile byte[] documentResult;
    private byte[] exportBytes;
    private InputManager inputManager;
    private final AtomicBoolean inputRemoved = new AtomicBoolean();

    @Override
    protected void onCreate(android.os.Bundle state) {
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        super.onCreate(state);
        inputManager = getSystemService(InputManager.class);
        if (inputManager != null) {
            inputManager.registerInputDeviceListener(this, new Handler(Looper.getMainLooper()));
        }
    }

    @Override
    protected void onDestroy() {
        if (inputManager != null) {
            inputManager.unregisterInputDeviceListener(this);
        }
        super.onDestroy();
    }

    @Override public void onInputDeviceAdded(int deviceId) {}
    @Override public void onInputDeviceChanged(int deviceId) {}
    @Override public void onInputDeviceRemoved(int deviceId) {
        inputRemoved.set(true);
    }

    public boolean takeInputRemoved() {
        return inputRemoved.getAndSet(false);
    }

    /** Physical pixel insets for any camera notch, including in immersive mode. */
    public int[] displayCutoutInsets() {
        WindowInsets insets = getWindow().getDecorView().getRootWindowInsets();
        DisplayCutout cutout = Build.VERSION.SDK_INT >= 28 && insets != null
            ? insets.getDisplayCutout()
            : null;
        return cutout == null ? new int[4] : new int[] {
            cutout.getSafeInsetLeft(), cutout.getSafeInsetTop(),
            cutout.getSafeInsetRight(), cutout.getSafeInsetBottom()
        };
    }

    public void openSaveDocument(int slot, boolean exporting, byte[] bytes) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                chooseDocument(slot, exporting, bytes);
            }
        });
    }

    private void chooseDocument(int slot, boolean exporting, byte[] bytes) {
        documentResult = null;
        exportBytes = exporting ? bytes : null;
        Intent intent = new Intent(exporting ? Intent.ACTION_CREATE_DOCUMENT : Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType(exporting ? "application/json" : "*/*");
        if (exporting) {
            intent.putExtra(Intent.EXTRA_TITLE, "Waldwinkel-Spielstand-" + slot + ".json");
        }
        try {
            startActivityForResult(intent, exporting ? SAVE_DOCUMENT : READ_DOCUMENT);
        } catch (RuntimeException failure) {
            exportBytes = null;
            documentResult = new byte[] {0};
        }
    }

    public byte[] takeSaveDocumentResult() {
        byte[] result = documentResult;
        if (result != null) {
            documentResult = null;
        }
        return result;
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != SAVE_DOCUMENT && request != READ_DOCUMENT) {
            return;
        }
        byte[] payload = exportBytes;
        exportBytes = null;
        Uri uri = data == null ? null : data.getData();
        if (result != RESULT_OK || uri == null) {
            documentResult = new byte[0];
            return;
        }
        new Thread(new Runnable() {
            @Override
            public void run() {
                transferDocument(uri, request, payload);
            }
        }, "Waldwinkel-Dokumente").start();
    }

    private void transferDocument(Uri uri, int request, byte[] payload) {
        try {
            if (request == SAVE_DOCUMENT) {
                try (OutputStream file = getContentResolver().openOutputStream(uri, "wt")) {
                    if (file == null || payload == null) {
                        throw new IOException("No save document");
                    }
                    file.write(payload);
                }
                documentResult = new byte[] {1};
            } else {
                try (InputStream file = getContentResolver().openInputStream(uri)) {
                    if (file == null) {
                        throw new IOException("No save document");
                    }
                    ByteArrayOutputStream output = new ByteArrayOutputStream();
                    output.write(1);
                    byte[] chunk = new byte[8192];
                    int count;
                    while ((count = file.read(chunk)) != -1) {
                        if (output.size() + count > MAX_SAVE_BYTES + 1) {
                            throw new IOException("Save document too large");
                        }
                        output.write(chunk, 0, count);
                    }
                    documentResult = output.toByteArray();
                }
            }
        } catch (IOException | RuntimeException failure) {
            documentResult = new byte[] {0};
        }
    }

    @Override
    public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        if (!focused) {
            return;
        }
        if (Build.VERSION.SDK_INT >= 30) {
            WindowInsetsController controller = getWindow().getInsetsController();
            if (controller != null) {
                controller.setSystemBarsBehavior(
                    WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                );
                controller.hide(WindowInsets.Type.systemBars());
            }
        } else {
            getWindow().getDecorView().setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_FULLSCREEN
                | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            );
        }
    }
}
