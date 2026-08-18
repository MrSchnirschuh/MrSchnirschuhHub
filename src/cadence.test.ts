import { describe, it, expect } from "vitest";
import {
  getDurationTotalMs,
  getIntervalMilliseconds,
  convertRateToDuration,
  convertDurationToRate,
  getEffectiveIntervalMs,
  getEffectiveClicksPerSecond,
  isDoubleClickSupported,
  formatDurationSummary,
} from "./cadence";

const baseDuration = {
  durationHours: 1,
  durationMinutes: 2,
  durationSeconds: 3,
  durationMilliseconds: 4,
};

describe("cadence", () => {
  it("computes duration total ms", () => {
    expect(getDurationTotalMs(baseDuration)).toBe(1 * 3600000 + 2 * 60000 + 3 * 1000 + 4);
  });

  it("returns interval ms", () => {
    expect(getIntervalMilliseconds("s")).toBe(1000);
    expect(getIntervalMilliseconds("m")).toBe(60000);
    expect(getIntervalMilliseconds("h")).toBe(3600000);
    expect(getIntervalMilliseconds("d")).toBe(86400000);
    expect(getIntervalMilliseconds("x" as unknown as "s")).toBe(1000);
  });

  it("converts rate to duration", () => {
    const result = convertRateToDuration({ ...baseDuration, clickSpeed: 2, clickInterval: "s", rateInputMode: "rate" });
    expect(result).not.toBeNull();
    expect(result!.durationMilliseconds).toBe(500);
  });

  it("returns null for invalid rate", () => {
    expect(convertRateToDuration({ ...baseDuration, clickSpeed: 0, clickInterval: "s", rateInputMode: "rate" })).toBeNull();
  });

  it("converts duration to rate", () => {
    const result = convertDurationToRate({ ...baseDuration, clickSpeed: 1, clickInterval: "s", rateInputMode: "duration" });
    expect(result).not.toBeNull();
    expect(result!.clickSpeed).toBeGreaterThan(0);
  });

  it("computes effective interval in rate mode", () => {
    const ms = getEffectiveIntervalMs({ ...baseDuration, clickSpeed: 2, clickInterval: "s", rateInputMode: "rate" });
    expect(ms).toBe(500);
  });

  it("computes effective interval in duration mode", () => {
    const ms = getEffectiveIntervalMs({ ...baseDuration, clickSpeed: 1, clickInterval: "s", rateInputMode: "duration" });
    expect(ms).toBe(getDurationTotalMs(baseDuration));
  });

  it("computes clicks per second", () => {
    expect(getEffectiveClicksPerSecond({ ...baseDuration, clickSpeed: 2, clickInterval: "s", rateInputMode: "rate" })).toBe(2);
  });

  it("detects double click support", () => {
    expect(isDoubleClickSupported({ ...baseDuration, clickSpeed: 10, clickInterval: "s", rateInputMode: "rate" })).toBe(true);
    expect(isDoubleClickSupported({ ...baseDuration, clickSpeed: 100, clickInterval: "s", rateInputMode: "rate" })).toBe(false);
  });

  it("formats duration summary", () => {
    expect(formatDurationSummary(baseDuration)).toBe("1h 2m 3s 4ms");
    expect(formatDurationSummary({ durationHours: 0, durationMinutes: 0, durationSeconds: 0, durationMilliseconds: 0 })).toBe("0ms");
  });
});
