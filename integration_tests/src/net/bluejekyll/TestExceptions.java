package net.bluejekyll;

public class TestExceptions {
    static void runTests() {
        System.out.println(">>>> Running " + TestExceptions.class.getName());
        TestExceptions.testThrowsSomething();
        TestExceptions.testThrowsSomethingMsg();
        TestExceptions.testCatchesSomething();
        System.out.println("<<<< " + TestExceptions.class.getName() + " tests succeeded");
    }

    public static void testThrowsSomething() {
        Exceptions exceptions = new Exceptions();

        String caught;
        try {
            exceptions.throwsSomething();
            caught = null;
        } catch (SomethingException e) {
            caught = e.getMessage();
        }

        if (caught == null) {
            throw new RuntimeException("no exception caught");
        } else {
            System.out.println("caught exception: " + caught);
        }
    }

    public static void testThrowsSomethingMsg() {
        Exceptions exceptions = new Exceptions();
        String expected = "Recieved Exception";

        String caught;
        try {
            exceptions.throwsSomething(expected);
            caught = null;
        } catch (SomethingException e) {
            caught = e.getMessage();
        }

        if (caught == null) {
            throw new RuntimeException("no exception caught");
        } else {
            System.out.println("caught exception: " + caught);
        }
    }

    public static void testCatchesSomething() {
        Exceptions exceptions = new Exceptions();

        SomethingException exception = exceptions.catchesSomething();

        if (!exception.getMessage().equals("iAlwaysThrow")) {
            throw new RuntimeException("no exception caught");
        }
    }
}