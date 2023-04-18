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
        test_invert();
        System.out.println("<<<< " + TestPrimitives.class.getName() + " tests succeeded");
    }

    static void test_void_void() {
        // Now construct the NativeClass

        NativePrimitives.voidVoid();
    }

    static void test_void_long() {
        NativePrimitives.voidLong(100);
    }

    static void test_void_long2() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.voidLong(100, 10);

        if (ret != 110) {
            throw new RuntimeException("Expected 110, got: " + ret);
        }
    }

    static void test_long_int_int() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.longIntInt(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {
            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_add_values_native() {
        NativePrimitives obj = new NativePrimitives();
        long ret = obj.addValuesNative(Integer.MAX_VALUE, Integer.MAX_VALUE);

        long expect = (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE;
        if (ret != expect) {

            throw new RuntimeException(
                    "Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }

    static void test_print_hello() {
        NativePrimitives.printHelloNativeStatic();

        NativePrimitives obj = new NativePrimitives();
        obj.printHelloNative();
    }

    static void test_call_dad() {
        NativePrimitives obj = new NativePrimitives();
        int expected = 732;
        int got = obj.callDadNative(expected);

        if (expected != got) {
            throw new RuntimeException("Expected " + expected + " got " + got);
        }
    }

    static void test_invert() {
        NativePrimitives obj = new NativePrimitives();
        if (obj.invert(true)) {
            throw new RuntimeException("Expected false");
        }
    }
}
