package net.bluejekyll;

public class NativeArrays {
    // get bytes of len length
    public static native void sendBytes(byte[] bytes);

    public static native byte[] getBytes(byte[] bytes);

    public static native byte[] newBytes();
}
