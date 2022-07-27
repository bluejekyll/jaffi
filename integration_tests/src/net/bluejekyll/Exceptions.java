package net.bluejekyll;

public class Exceptions {
    public native void throwsSomething() throws SomethingException;
    public native void throwsSomething(String msg) throws SomethingException;
}
