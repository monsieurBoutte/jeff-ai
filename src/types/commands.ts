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

interface Weather {
  id: number;
  main: string;
  description: string;
  icon: string;
}

interface CurrentWeather {
  dt: number;
  sunrise: number;
  sunset: number;
  temp: number;
  feels_like: number;
  pressure: number;
  humidity: number;
  dew_point: number;
  uvi: number;
  clouds: number;
  visibility: number;
  wind_speed: number;
  wind_deg: number;
  weather: Weather[];
}

interface HourlyWeather extends CurrentWeather {
  wind_gust?: number;
  pop: number;
  rain?: {
    '1h': number;
  };
}

interface DailyTemp {
  day: number;
  min: number;
  max: number;
  night: number;
  eve: number;
  morn: number;
}

interface DailyFeelsLike {
  day: number;
  night: number;
  eve: number;
  morn: number;
}

interface DailyWeather {
  dt: number;
  sunrise: number;
  sunset: number;
  moonrise: number;
  moonset: number;
  moon_phase: number;
  summary: string;
  temp: DailyTemp;
  feels_like: DailyFeelsLike;
  pressure: number;
  humidity: number;
  dew_point: number;
  wind_speed: number;
  wind_deg: number;
  wind_gust: number;
  weather: Weather[];
  clouds: number;
  pop: number;
  rain?: number;
  uvi: number;
}

export interface WeatherResponse {
  data: {
    lat: number;
    lon: number;
    timezone: string;
    timezone_offset: number;
    current: CurrentWeather;
    hourly: HourlyWeather[];
    daily: DailyWeather[];
  };
}
