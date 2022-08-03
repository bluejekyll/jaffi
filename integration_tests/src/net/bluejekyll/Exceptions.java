package net.bluejekyll;

public class Exceptions {
    public native void throwsSomething() throws SomethingException;

    public native void throwsSomething(String msg) throws SomethingException;

    public native SomethingException catchesSomething();

    public void iAlwaysThrow() throws SomethingException {
        throw new SomethingException("iAlwaysThrow");
    }
}
