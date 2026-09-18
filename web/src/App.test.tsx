import {createRoot} from "react-dom/client";
import {act} from "react";
import {App} from "./App";
import {beforeEach,describe,expect,it,vi} from "vitest";

beforeEach(() => {
  const storage = () => {
    const values = new Map<string,string>();
    return {getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key,value), removeItem: (key: string) => values.delete(key), clear: () => values.clear()};
  };
  vi.stubGlobal("localStorage",storage());
  vi.stubGlobal("sessionStorage",storage());
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT",true);
  localStorage.clear();sessionStorage.clear();
});
describe("App",()=>{
  it("renders an editable unauthenticated composer with a truthful connection action",()=>{
    const el=document.createElement("div");document.body.appendChild(el);const root=createRoot(el);
    act(()=>root.render(<App/>));
    expect(el.textContent).toMatch(/What should happen/);
    expect(el.textContent).toMatch(/Run bf to open an authenticated workbench/);
    expect(el.querySelector("textarea")?.disabled).toBe(false);
    act(()=>root.unmount());el.remove();
  });
});
