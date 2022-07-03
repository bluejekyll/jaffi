
package net.bluejekyll;

/**
 * Simple test classes. This wires up the dylib, then checks all the inerfaces
 * ensuring that data is
 * being passed from Java to Rust and vice versa correctly.
 */
public class TestCases {
    public static void main(String[] args) {
        System.out.println("Running tests");

        String lib = System.getenv("JAFFI_LIB");
        System.loadLibrary(lib);
        System.out.printf("loadLibrary succeeded for %s%n", lib);

        System.out.printf("running tests %s%n", lib);
        test_void_void();
        test_void_long();
        test_void_long2();
        test_long_int_int();
        test_add_values_native();
        test_print_hello();
        test_call_dad();
    }

    static void test_void_void() {
        // Now construct the NativeClass

        NativeClass.void_void();
    }

    static void test_void_long() {
        NativeClass.void_long(100);
    }

    static void test_void_long2() {
        NativeClass obj = new NativeClass();
        long ret = obj.void_long(100, 10);

        if (ret != 110) {
            throw new RuntimeException("Expected 110, got: " + ret);
        }
    }

    static void test_long_int_int() {
        NativeClass obj = new NativeClass();
        long ret = obj.long_int_int(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {
            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_add_values_native() {
        NativeClass obj = new NativeClass();
        long ret = obj.add_values_native(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {

            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_print_hello() {
        NativeClass.print_hello_native_static();

        NativeClass obj = new NativeClass();
        obj.print_hello_native();
    }

    static void test_call_dad() {
        NativeClass obj = new NativeClass();
        int expected = 732;
        int got = obj.call_dad_native(expected);

        if (expected != got) {
            throw new RuntimeException("Expected " + expected + " got " + got);
        }
    }
}
