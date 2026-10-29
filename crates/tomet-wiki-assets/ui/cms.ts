import initWasm, {
  GitHubCms as RawGitHubCms,
  listConflicts as rawListConflicts,
  resolveConflict as rawResolveConflict,
} from "@tomet/cms/pkg";
import { getWasmBytes } from "virtual:tomet-cms-wasm";

export interface ConflictInfo {
  index: number;
  kind: "block" | "inline" | "scalar";
  span: { start: number; end: number };
  a: string;
  b: string;
}

export interface SaveResult {
  revision: string;
  hasConflicts: boolean;
  warnings?: Array<{ code: string; path: string; message: string }>;
}

let wasmInitPromise: Promise<void> | null = null;

export async function initCms(): Promise<void> {
  if (!wasmInitPromise) {
    wasmInitPromise = (async () => {
      const bytes = await getWasmBytes();
      await initWasm({ module_or_path: bytes });
    })();
  }
  return wasmInitPromise;
}

export function listConflicts(content: string): ConflictInfo[] {
  try {
    return rawListConflicts(content);
  } catch (err) {
    console.warn("Failed to list conflicts:", err);
    return [];
  }
}

export function resolveConflict(content: string, index: number, side: "a" | "b"): string {
  return rawResolveConflict(content, index, side);
}

export function resolveRepoPath(relPath: string, pathPrefix?: string): string {
  const cleanRel = relPath.replace(/^\/+/, "");
  if (!pathPrefix) return cleanRel;
  const cleanPrefix = pathPrefix.replace(/^\/+|\/+$/g, "");
  return cleanPrefix ? `${cleanPrefix}/${cleanRel}` : cleanRel;
}

export async function fetchFileFromGitHub(
  owner: string,
  repo: string,
  path: string,
  branch = "main",
  token?: string | null,
  apiBaseUrl = "https://api.github.com",
): Promise<{ content: string; sha: string }> {
  const cleanPath = path.replace(/^\/+/, "");
  const url = `${apiBaseUrl}/repos/${owner}/${repo}/contents/${cleanPath}?ref=${encodeURIComponent(branch)}`;
  const headers: Record<string, string> = {
    Accept: "application/vnd.github.v3+json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const res = await fetch(url, { headers });
  if (res.status === 404) {
    throw new Error(`File '${cleanPath}' does not exist on branch '${branch}'`);
  }
  if (!res.ok) {
    throw new Error(`GitHub API error (${res.status} ${res.statusText})`);
  }

  const data = await res.json();
  const rawBase64 = (data.content || "").replace(/\s/g, "");
  const binaryString = atob(rawBase64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const content = new TextDecoder("utf-8").decode(bytes);

  return {
    content,
    sha: data.sha,
  };
}

export async function saveFileViaCms(
  owner: string,
  repo: string,
  path: string,
  branch: string,
  content: string,
  message: string,
  baseRev: string | null,
  token: string,
  apiBaseUrl = "https://api.github.com",
): Promise<SaveResult> {
  await initCms();

  const cleanPath = path.replace(/^\/+/, "");
  const client = new RawGitHubCms({
    owner,
    repo,
    branch: branch || "main",
    token,
    baseUrl: apiBaseUrl,
  });

  if (baseRev) {
    // Edit existing file with 3-way AST merge
    const result = await client.editFile({
      path: cleanPath,
      baseRev,
      content,
      message,
    });
    return result as SaveResult;
  } else {
    // Create new file
    const result = await client.createFile({
      path: cleanPath,
      content,
      message,
    });
    return result as SaveResult;
  }
}
