package net.bluejekyll;

public class TestArrays {
    static void runTests() {
        System.out.println(">>>> Running " + TestStrings.class.getName());
        TestArrays.testSendBytes();
        TestArrays.testGetBytes();
        TestArrays.testNewBytes();
        System.out.println("<<<< " + TestStrings.class.getName() + " tests succeeded");
    }

    static void testSendBytes() {
        byte[] bytes = java.util.HexFormat.of().parseHex("CAFEBABE");
        NativeArrays.sendBytes(bytes);
    }

    static void testGetBytes() {
        byte[] expect = java.util.HexFormat.of().parseHex("CAFEBABE");
        byte[] got = NativeArrays.getBytes(expect);

        if (!java.util.Arrays.equals(got, expect)) {
            throw new RuntimeException("Expected " + expect + " got " + got);
        }
    }

    static void testNewBytes() {
        byte[] expect = java.util.HexFormat.of().parseHex("CAFEBABE");
        byte[] got = NativeArrays.newBytes();

        if (!java.util.Arrays.equals(got, expect)) {
            throw new RuntimeException("Expected " + expect + " got " + got);
        }
    }
}
