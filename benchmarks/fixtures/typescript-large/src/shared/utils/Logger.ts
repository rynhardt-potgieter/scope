/** Log severity levels */
export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error',
}

/** Structured log entry */
interface LogEntry {
  level: LogLevel;
  message: string;
  context: string;
  timestamp: Date;
  data?: Record<string, unknown>;
}

/** Application logger with structured output and context tagging */
export class Logger {
  private context: string;
  private minLevel: LogLevel;

  constructor(context: string, minLevel: LogLevel = LogLevel.INFO) {
    this.context = context;
    this.minLevel = minLevel;
  }

  /** Log an informational message */
  info(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.INFO, message, data);
  }

  /** Log a warning */
  warn(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.WARN, message, data);
  }

  /** Log an error with optional stack trace */
  error(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.ERROR, message, data);
  }

  /** Log a debug message — only shown when minLevel is DEBUG */
  debug(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.DEBUG, message, data);
  }

  /** Creates a child logger with a more specific context */
  child(subContext: string): Logger {
    return new Logger(`${this.context}.${subContext}`, this.minLevel);
  }

  private log(level: LogLevel, message: string, data?: Record<string, unknown>): void {
    if (this.shouldLog(level)) {
      const entry: LogEntry = {
        level,
        message,
        context: this.context,
        timestamp: new Date(),
        data,
      };
      const output = JSON.stringify(entry);
      if (level === LogLevel.ERROR) {
        console.error(output);
      } else {
        console.log(output);
      }
    }
  }

  private shouldLog(level: LogLevel): boolean {
    const levels = [LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN, LogLevel.ERROR];
    return levels.indexOf(level) >= levels.indexOf(this.minLevel);
  }
}
