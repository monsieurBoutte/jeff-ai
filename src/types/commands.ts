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

interface LocalName {
  ar?: string; // Arabic
  be?: string; // Belarusian
  bg?: string; // Bulgarian
  en: string; // English
  [key: string]: string | undefined; // Allow for other language codes
}

export interface WeatherLocationResult {
  name: string;
  lat: number;
  lon: number;
  country: string;
  state: string;
  local_names?: LocalName;
}
