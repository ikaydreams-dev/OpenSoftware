export const englishCodeLanguage = {
  id: "englishcode",
  extensions: [".eng"],
  aliases: ["EnglishCode", "eng"],
  mimetypes: ["text/x-englishcode"],
};

export const englishCodeConfig = {
  comments: {
    lineComment: "//",
  },
  brackets: [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
  ],
  autoClosingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: "(", close: ")" },
    { open: '"', close: '"' },
  ],
};

export const englishCodeTokens = {
  keywords: [
    "create",
    "database",
    "collection",
    "collections",
    "called",
    "named",
    "insert",
    "into",
    "select",
    "from",
    "update",
    "delete",
    "set",
    "let",
    "make",
    "store",
    "save",
    "show",
    "get",
    "find",
    "fetch",
    "add",
    "put",
    "remove",
    "change",
    "modify",
    "with",
    "and",
    "where",
    "all",
    "these",
    "in",
    "it",
    "to",
    "is",
    "a",
    "an",
    "the",
  ],

  typeKeywords: ["true", "false", "null"],

  tokenizer: {
    root: [
      // Keywords
      [
        /\b(create|database|collection|collections|called|named|insert|into|select|from|update|delete|set|let|make|store|save|show|get|find|fetch|add|put|remove|change|modify|with|and|where|all|these|in|it|to|is|a|an|the)\b/,
        "keyword",
      ],

      // Boolean and null
      [/\b(true|false|null)\b/, "constant.language"],

      // Numbers
      [/\d+\.?\d*/, "number"],

      // Strings
      [/"([^"\\]|\\.)*$/, "string.invalid"],
      [/"/, { token: "string.quote", bracket: "@open", next: "@string" }],

      // Identifiers
      [/[a-zA-Z_]\w*/, "identifier"],

      // Whitespace
      [/[ \t\r\n]+/, "white"],
    ],

    string: [
      [/[^\\"]+/, "string"],
      [/\\./, "string.escape.invalid"],
      [/"/, { token: "string.quote", bracket: "@close", next: "@pop" }],
    ],
  },
};
