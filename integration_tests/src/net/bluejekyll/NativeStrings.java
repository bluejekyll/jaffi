package net.bluejekyll;

public class NativeStrings {
    /// Test passing a string to Rust
    public native void eatString(String str);

    /// Return a String from Java to Rust
    public String returnString() {
        return "I am a return string";
    }
}
