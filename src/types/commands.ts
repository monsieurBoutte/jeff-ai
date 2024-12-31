export interface RefineTextResponse {
  refinedText: string;
  explanation?: string | undefined;
}

export interface MarkdownResponse {
  markdown: string;
}

export interface Refinement {
  id: string;
  userId: string;
  createdAt: string;
  explanation?: string;
  originalText: string;
  originalTextWordCount: number;
  refinedText: string;
  refinedTextWordCount: number;
  updatedAt: string;
  vector: number[];
}
