package net.bluejekyll;

public class TestStrings {
    static void runTests() {
        System.out.println(">>>> Running " + TestStrings.class.getName());
        TestStrings.testEatString();
        TestStrings.testTieOffString();
        TestStrings.testReturnStringFromJava();
        System.out.println("<<<< " + TestStrings.class.getName() + " tests succeeded");
    }

    static void testEatString() {
        NativeStrings strings = new NativeStrings();
        strings.eatString("everyone loves rust: i❤🦀");
    }

    static void testTieOffString() {
        String expected = "does this round trip? i❤🦀";
        NativeStrings strings = new NativeStrings();
        String got = strings.tieOffString(expected);

        if (!expected.equals(got)) {
            throw new RuntimeException("expected " + expected + " got " + got);
        }
    }

    static void testReturnStringFromJava() {
        String expected = NativeStrings.retString + " and ☕️";
        NativeStrings strings = new NativeStrings();
        String got = strings.returnStringNative(" and ☕️");

        if (!expected.equals(got)) {
            throw new RuntimeException("expected " + expected + " got " + got);
        }
    }
}
