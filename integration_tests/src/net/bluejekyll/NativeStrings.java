package net.bluejekyll;

public class NativeStrings {
    public static String retString = "I am a return string and i❤🦀";
    private final String message;

    public NativeStrings() {
        this(retString);
    }

    public NativeStrings(String str) {
        this.message = str;
    }

    public static native NativeStrings ctor(String s);

    // Test passing a string to Rust
    public native void eatString(String str);

    // This just roundtrips the string
    public native String tieOffString(String str);

    public native String returnStringNative(String append);

    // Return a String from Java to Rust
    public String returnString(String append) {
        return message + append;
    }
}
