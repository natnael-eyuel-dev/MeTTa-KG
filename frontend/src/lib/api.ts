import { ImportDataResponse, Token, ExploreDetail, Mm2Input } from "./types";
import { CSVParserParameters } from "~/types";
import { quoteFromBytes } from "./utils";

export const API_URL =
  import.meta.env.VITE_BACKEND_URL || "http://localhost:8000";

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem("rootToken");
  const headers = {
    ...options.headers,
    // Authorization: "200003ee-c651-4069-8b7f-2ad9fb46c3ab",
    ...(token && { Authorization: token }),
  };

  console.log(`token: ${token}`);

  const response = await fetch(`${API_URL}${url}`, { ...options, headers });
  if (!response.ok) {
    let errorMessage = response.statusText;
    const cloned = response.clone();
    try {
      const errorData = await cloned.json();
      errorMessage = errorData.message || errorMessage;
    } catch {
      try {
        errorMessage = (await response.text()) || errorMessage;
      } catch {
        // ignore
      }
    }
    throw new Error(errorMessage);
  }
  try {
    const res = await response.json();
    return res;
  } catch (e) {
    if (
      e instanceof SyntaxError &&
      e.message.includes("Unexpected end of JSON input")
    ) {
      return null as T;
    }
    throw e;
  }
}

export const transform = (path: string, transformation: Mm2Input) => {
  const patterns = Array.isArray(transformation.pattern)
    ? transformation.pattern
    : [transformation.pattern];
  const templates = Array.isArray(transformation.template)
    ? transformation.template
    : [transformation.template];

  return request<boolean>(`/spaces/transform${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ patterns, templates }),
  })
    .then((result) => {
      return result;
    })
    .catch((error) => {
      throw error;
    });
};

export const readSpace = (path: string) => {
  return request<string>(`/spaces${path}`);
};

export const getAllTokens = () => {
  return request<Token[]>("/tokens");
};

export const getToken = () => {
  return request<Token>("/token");
};

export const createFromCSV = (file: File, params: CSVParserParameters) => {
  const formData = new FormData();
  formData.append("file", file);
  const url = new URL(`${API_URL}/translations/csv`);
  url.search = new URLSearchParams(
    params as any /* eslint-disable-line @typescript-eslint/no-explicit-any */
  ).toString();

  return fetch(url.toString(), {
    method: "POST",
    body: formData,
    headers: {
      Authorization: `${localStorage.getItem("rootToken")}`,
    },
  }).then((response) => response.json());
};

export const createFromNT = (file: File) => {
  const formData = new FormData();
  formData.append("file", file);

  return fetch(`${API_URL}/translations/nt`, {
    method: "POST",
    body: formData,
    headers: {
      Authorization: `${localStorage.getItem("rootToken")}`,
    },
  }).then((response) => response.json());
};

export const createFromJsonLd = (file: File) => {
  const formData = new FormData();
  formData.append("file", file);

  return fetch(`${API_URL}/translations/jsonld`, {
    method: "POST",
    body: formData,
    headers: {
      Authorization: `${localStorage.getItem("rootToken")}`,
    },
  }).then((response) => response.json());
};

export const createFromN3 = (file: File) => {
  const formData = new FormData();
  formData.append("file", file);

  return fetch(`${API_URL}/translations/n3`, {
    method: "POST",
    body: formData,
    headers: {
      Authorization: `${localStorage.getItem("rootToken")}`,
    },
  }).then((response) => response.json());
};

export async function isPathClear(path: string): Promise<boolean> {
  try {
    const cleanPath = path.replace(/^\/+|\/+$/g, "");

    const requestBody = {
      pattern: "$x",
      token: "",
    };

    // TODO: use requests function and return value from it
    const _response = await fetch(`${API_URL}/spaces/explore${cleanPath}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(requestBody),
    });

    return true;
  } catch {
    return true;
  }
}

export async function importData(
  type: string,
  data: any /* eslint-disable-line @typescript-eslint/no-explicit-any */ = null,
  format: string = "metta"
): Promise<ImportDataResponse> {
  try {
    switch (type) {
      case "text": {
        // Use the upload endpoint like the pattern you showed
        const url = `${API_URL}/upload/${encodeURIComponent("$x")}/${encodeURIComponent("$x")}?format=${encodeURIComponent(format)}`;
        const res = await fetch(url, {
          method: "POST",
          headers: {
            "Content-Type": "text/plain",
            ...(localStorage.getItem("rootToken") && {
              Authorization: localStorage.getItem("rootToken"),
            }),
          },
          body: data as string,
        });

        const text = await res.text();

        if (!res.ok) {
          return {
            status: "error",
            message: text || res.statusText,
          };
        }

        return {
          status: "success",
          data: text,
          message: "Text imported successfully",
        };
      }

      case "file":
        return {
          status: "error",
          message: "File upload not implemented yet",
        };

      default:
        return {
          status: "error",
          message: `Unsupported import type: ${type}`,
        };
    }
  } catch (error) {
    return {
      status: "error",
      message: error instanceof Error ? error.message : "Failed to import data",
    };
  }
}

export const uploadTextToSpace = (
  path: string,
  data: string
): Promise<string> => {
  return request<string>(`/spaces/upload${path}`, {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: data,
  });
};

export const importSpace = (path: string, uri: string) => {
  return request<boolean>(`/spaces/import${path}?uri=${uri}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });
};

export const fetchTokens = async (token: string | null): Promise<Token[]> => {
  localStorage.setItem("rootToken", token ?? "");
  if (!token) return [];
  return request<Token[]>("/tokens", {
    method: "GET",
    headers: { Authorization: token },
  });
};

export const createToken = async (
  root: string | null,
  description: string,
  namespace: string,
  read: boolean,
  write: boolean,
  shareRead: boolean,
  shareWrite: boolean,
  shareShare: boolean
): Promise<Token> => {
  if (!root) throw new Error("No root token");

  const newToken: Token = {
    id: 0,
    code: "",
    description: description,
    namespace: namespace,
    creation_timestamp: new Date().toISOString().split("Z")[0],
    permission_read: read,
    permission_write: write,
    permission_share_read: shareRead,
    permission_share_write: shareWrite,
    permission_share_share: shareShare,
    parent: 0,
  };

  return request<Token>("/tokens", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: root,
    },
    body: JSON.stringify(newToken),
  });
};

export const refreshCodes = async (
  root: string | null,
  tokenIds: number[]
): Promise<Token[]> => {
  if (!root) return [];
  const promises = tokenIds.map((id) =>
    request<Token>(`/tokens/${id}`, {
      method: "POST",
      headers: { Authorization: root },
    })
  );
  return Promise.all(promises);
};

export const deleteToken = (root: string | null, token_id: number) => {
  if (!root) throw new Error("No root token");
  return request(`/tokens/${token_id}`, {
    method: "DELETE",
    headers: { Authorization: root },
  });
};

export const deleteTokens = (root: string | null, token_ids: number[]) => {
  if (!root) throw new Error("No root token");
  return request<number>("/tokens", {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
      Authorization: root,
    },
    body: JSON.stringify(token_ids),
  });
};

export const exploreSpace = (
  path: string,
  pattern: string,
  token: Uint8Array | Array<number>
) => {
  if (token instanceof Array) {
    token = Uint8Array.from(token);
  }

  return request<ExploreDetail[]>(`/spaces/explore${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      pattern,
      token: quoteFromBytes(token),
    }),
  });
};

export const exportSpace = async (
  path: string,
  exportInput: Mm2Input
): Promise<string> => {
  if (exportInput.pattern && typeof exportInput.pattern !== "string") {
    exportInput.pattern = exportInput.pattern[0];
  }

  if (exportInput.template && typeof exportInput.template !== "string") {
    exportInput.template = exportInput.template[0];
  }

  return request<string>(`/spaces/export${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(exportInput),
  });
};

export const clearSpace = (path: string) => {
  return request<boolean>(`/spaces/clear${path}?expr=$x`, {
    method: "POST",
  });
};
