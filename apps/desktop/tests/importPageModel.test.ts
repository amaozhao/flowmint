import type { RemoteImportCandidate } from "../src/api/import";
import {
  defaultGithubCandidateDecision,
  importProjectPathRequiredForSource,
  isValidPublicGithubUrl,
  normalizeRemoteDestinationId,
} from "../src/pages/importPageModel";

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

function candidate(overrides: Partial<RemoteImportCandidate> = {}): RemoteImportCandidate {
  return {
    candidateId: "skill:research-helper:abc",
    id: "research-helper",
    assetType: "skill",
    confidence: "high",
    source: {
      provider: "public-github",
      owner: "example",
      repo: "agent-assets",
      refName: "main",
      commitSha: "abc123",
      rootPath: "",
      canonicalUrl: "https://github.com/example/agent-assets/tree/main",
    },
    sourcePaths: [".codex/skills/research-helper/SKILL.md"],
    defaultDestinationId: "research-helper",
    collision: null,
    warnings: [],
    importable: true,
    ...overrides,
  };
}

assert(
  importProjectPathRequiredForSource("local-tool", "project"),
  "local project import requires a project path",
);
assert(
  !importProjectPathRequiredForSource("public-github", "project"),
  "GitHub import can scan before a current project is selected",
);
assert(isValidPublicGithubUrl("https://github.com/example/repo"), "plain public GitHub repo URL is valid");
assert(
  isValidPublicGithubUrl(" https://github.com/example/repo/tree/main/.codex/skills/demo "),
  "GitHub tree URL is valid after trimming",
);
assert(!isValidPublicGithubUrl("https://example.com/example/repo"), "non-GitHub URL is invalid");
assert(
  normalizeRemoteDestinationId(" Research Helper! ") === "research-helper",
  "remote destination id is normalized to safe asset id",
);
assert(
  defaultGithubCandidateDecision(candidate()) === "skip",
  "remote candidates default to skip so users explicitly choose what to import",
);
assert(
  defaultGithubCandidateDecision(candidate({ collision: { assetRef: "skill:research-helper", libraryPath: "/tmp" } })) === "skip",
  "colliding remote candidates default to skip",
);
assert(
  defaultGithubCandidateDecision(candidate({ importable: false })) === "skip",
  "unimportable remote candidates default to skip",
);
