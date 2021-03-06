package net.bluejekyll;

public class NativePrimitives extends ParentClass {
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

    public native int call_dad_native(int arg1);

    public native java.io.File unsupported(java.io.File file);

    public java.io.File unsupported_method(java.io.File file) {
        // does nothing, this is a compilation check
        return file;
    }

    public Unsupported unsupported_return() {
        // does nothing, this is a compilation check
        return new Unsupported();
    }

    public native Unsupported2 unsupported_return_native();
}