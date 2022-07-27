package net.bluejekyll;

public class SomethingException extends Exception {
    public SomethingException() {
        super();
    }

    public SomethingException(String msg) {
        super(msg);
    }

    public SomethingException(String msg, Throwable cause) {
        super(msg, cause);
    }

    public SomethingException(Throwable cause) {
        super(cause);
    }
}