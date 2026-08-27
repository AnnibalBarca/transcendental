import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en/translation.json";
import fr from "./locales/fr/translation.json";
import es from "./locales/es/translation.json";
import sr from "./locales/sr/translation.json";
import de from "./locales/de/translation.json";
import it from "./locales/it/translation.json";

const resources = {
	en: { translation: en },
	fr: { translation: fr },
	es: { translation: es },
	sr: { translation: sr },
	de: { translation: de },
	it: { translation: it },
};

const SUPPORTED_LANGUAGES = ["en", "fr", "es", "sr", "de", "it"];
const STORAGE_KEY = "i18n-lang";

function getSavedLanguage(): string | null {
	if (typeof window === "undefined") return null;
	const saved = window.localStorage.getItem(STORAGE_KEY);
	if (saved && SUPPORTED_LANGUAGES.includes(saved)) return saved;

	const browserLang = window.navigator.language?.slice(0, 2).toLowerCase();
	if (browserLang && SUPPORTED_LANGUAGES.includes(browserLang)) return browserLang;

	return null;
}

function saveLanguage(lng: string) {
	if (typeof window === "undefined") return;
	window.localStorage.setItem(STORAGE_KEY, lng);
}

const savedLanguage = getSavedLanguage();

i18n
	.use(initReactI18next)
	.init({
		resources,
		lng: savedLanguage || undefined,
		fallbackLng: "en",
		supportedLngs: SUPPORTED_LANGUAGES,
		debug: false,
		interpolation: {
			escapeValue: false,
		},
	});

i18n.on("languageChanged", saveLanguage);

export default i18n;
