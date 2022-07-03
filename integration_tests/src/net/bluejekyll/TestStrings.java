package net.bluejekyll;

public class TestStrings {
    static void runTests() {
        System.out.println(">>>> Running " + TestStrings.class.getName());
        TestStrings.testEatString();
        System.out.println("<<<< " + TestStrings.class.getName() + " tests succeeded");
    }

    static void testEatString() {
        NativeStrings strings = new NativeStrings();
        strings.eatString("everyone loves rust: i❤🦀");
    }
}
