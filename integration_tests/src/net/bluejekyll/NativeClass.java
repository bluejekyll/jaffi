package net.bluejekyll;

public class NativeClass {
    // basic test
    public static native void void_void();

    // a parameter
    public static native void void_long(long foo);

    // a duplicate function name
    public native long void_long(long foo, int bar);

    // a return type
    public native long long_int_int(int foo, int bar);
}