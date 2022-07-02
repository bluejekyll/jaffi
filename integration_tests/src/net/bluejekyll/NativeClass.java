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

    // a native method that internally calls the object method add_values
    public native long add_values_native(int arg1, int arg2);

    public long add_values(int arg1, int arg2) {
        return (long) arg1 + (long) arg2;
    }

    public static native void print_hello_native_static();

    public native void print_hello_native();

    public static void print_hello() {
        System.out.println("hello!");
    }
}