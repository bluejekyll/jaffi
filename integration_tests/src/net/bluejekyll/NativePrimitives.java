package net.bluejekyll;

public class NativePrimitives extends ParentClass {
    // basic test
    public static native void voidVoid();

    // a parameter
    public static native void voidLong(long foo);

    // a duplicate function name
    public native long voidLong(long foo, int bar);

    // a return type
    public native long longIntInt(int foo, int bar);

    // a native method that internally calls the object method add_values
    public native long addValuesNative(int arg1, int arg2);

    public long addValues(int arg1, int arg2) {
        return (long) arg1 + (long) arg2;
    }

    public static native void printHelloNativeStatic();

    public native void printHelloNative();

    public static void printHello() {
        System.out.println("hello!");
    }

    public native int callDadNative(int arg1);

    public native boolean invert(boolean arg);

    public native java.io.File unsupported(java.io.File file);

    public java.io.File unsupportedMethod(java.io.File file) {
        // does nothing, this is a compilation check
        return file;
    }

    public Unsupported unsupportedReturn() {
        // does nothing, this is a compilation check
        return new Unsupported();
    }

    public native Unsupported2 unsupportedReturnNative();
}