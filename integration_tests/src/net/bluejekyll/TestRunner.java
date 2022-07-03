
package net.bluejekyll;

/**
 * Simple test classes. This wires up the dylib, then checks all the inerfaces
 * ensuring that data is
 * being passed from Java to Rust and vice versa correctly.
 */
public class TestRunner {
    public static void main(String[] args) {
        System.out.println("Running tests");

        String lib = System.getenv("JAFFI_LIB");
        System.loadLibrary(lib);
        System.out.printf("loadLibrary succeeded for %s%n", lib);

        System.out.println("Starting test run");
        TestPrimitives.runTests();
        TestStrings.runTests();
        System.out.println("All tests succeeded");
    }

}
