package net.bluejekyll;

public class TestExceptions {
    static void runTests() {
        System.out.println(">>>> Running " + TestExceptions.class.getName());
        TestExceptions.testThrowsSomething();
        TestExceptions.testThrowsSomethingMsg();
        System.out.println("<<<< " + TestExceptions.class.getName() + " tests succeeded");
    }

    public static void testThrowsSomething() {
        Exceptions exceptions = new Exceptions();

        boolean caught;
        try {
            exceptions.throwsSomething();
            caught = false;
        } catch (SomethingException e) {
            caught = true;
        }

        if (!caught) {
            throw new RuntimeException("no exception caught");
        } else {
            System.out.println("caught exception");
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
}