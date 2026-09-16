import { createRoot } from "react-dom/client";
import { act } from "react";
import { App } from "./App";
import { describe, expect, it } from "vitest";

describe("App", () => {
  it("renders the composer", () => {
    const el = document.createElement("div");
    document.body.appendChild(el);
    act(() => {
      createRoot(el).render(<App />);
    });
    expect(el.textContent).toMatch(/What should happen/);
    expect(el.querySelector("textarea")).toBeTruthy();
  });
});
