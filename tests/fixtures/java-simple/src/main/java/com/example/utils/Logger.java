package com.example.utils;

/**
 * Simple logging utility.
 */
public class Logger {

    private final String prefix;

    public Logger(String prefix) {
        this.prefix = prefix;
    }

    public void info(String message) {
        System.out.println("[INFO] " + prefix + ": " + message);
    }

    public void error(String message) {
        System.out.println("[ERROR] " + prefix + ": " + message);
    }

    public static Logger create(String name) {
        return new Logger(name);
    }
}
