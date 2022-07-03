package net.bluejekyll;

public class TestPrimitives {
    static void runTests() {
        System.out.println(">>>> Running " + TestPrimitives.class.getName());
        test_void_void();
        test_void_long();
        test_void_long2();
        test_long_int_int();
        test_add_values_native();
        test_print_hello();
        test_call_dad();
        System.out.println("<<<< " + TestPrimitives.class.getName() + " tests succeeded");
    }

    static void test_void_void() {
        // Now construct the NativeClass

        NativePrimitives.void_void();
    }

    static void test_void_long() {
        NativePrimitives.void_long(100);
    }

    static void test_void_long2() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.void_long(100, 10);

        if (ret != 110) {
            throw new RuntimeException("Expected 110, got: " + ret);
        }
    }

    static void test_long_int_int() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.long_int_int(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {
            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_add_values_native() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.add_values_native(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {

            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_print_hello() {
        NativePrimitives.print_hello_native_static();

        NativePrimitives obj = new NativePrimitives();
        obj.print_hello_native();
    }

    static void test_call_dad() {
        NativePrimitives obj = new NativePrimitives();
        int expected = 732;
        int got = obj.call_dad_native(expected);

        if (expected != got) {
            throw new RuntimeException("Expected " + expected + " got " + got);
        }
    }
}
