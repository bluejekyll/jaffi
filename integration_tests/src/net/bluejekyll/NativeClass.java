package net.bluejekyll;

public class NativeClass {
    // basic test
    public static native void test_void_void();

    // a parameter
    public static native void test_void_long(long foo);

    // a duplicate function name
    public native void test_void_long(long foo, long bar);

    // a return type
    public native long test_long_long(long foo, long bar);
}