import { createMemo, createSignal } from "solid-js";

const [rootToken, _setRootToken] = createSignal<string | null>(
  localStorage.getItem("rootToken")
);

// Initialize from localStorage
const storedNamespace = localStorage.getItem("tokenNamespace");
const initialNamespace = storedNamespace ? JSON.parse(storedNamespace) : [""];

const [tokenRootNamespace, setTokenRootNamespace] =
  createSignal<string[]>(initialNamespace);
const [namespace, setNamespace] = createSignal<string[]>(initialNamespace);

// New signal for configuration status
const [isConfigured, setIsConfigured] = createSignal(false);

export {
  rootToken,
  tokenRootNamespace,
  namespace,
  setNamespace,
  setTokenRootNamespace,
  isConfigured,
};

export const setRootToken = (token: string | null) => {
  localStorage.setItem("rootToken", token ?? "");
  _setRootToken(token);

  if (!token) {
    localStorage.removeItem("tokenNamespace");
    setTokenRootNamespace([""]);
    setNamespace([""]);
  }
};

export const formatedNamespace = createMemo(() => {
  if (namespace().length <= 1) return "/";
  return namespace().join("/");
});

export const checkConfiguration = async () => {
  try {
    const res = await fetch("/api/tokens");

    if (res.ok || res.status === 401) {
      setIsConfigured(true);
      return true;
    }

    setIsConfigured(false);
    return false;
  } catch {
    setIsConfigured(false);
    return false;
  }
};
