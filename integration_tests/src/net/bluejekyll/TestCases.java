
package net.bluejekyll;

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
            throw new RuntimeException("Expected " + (long) Integer.MAX_VALUE + (long) Integer.MAX_VALUE + ", got: " + ret);
        }
    }
}
