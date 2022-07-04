package net.bluejekyll;

public class NativeStrings {
    public static String retString = "I am a return string and i❤🦀";

    // Test passing a string to Rust
    public native void eatString(String str);

    // This just roundtrips the string
    public native String tieOffString(String str);

    public native String returnStringNative(String append);

    // Return a String from Java to Rust
    public String returnString(String append) {
        return retString + append;
    }
}
