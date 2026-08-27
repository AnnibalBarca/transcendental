import { StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import "@/assets/styles/globals.css";
import i18n from "./i18n";
import App from "./App.tsx";

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<I18nextProvider i18n={i18n}>
			<Suspense fallback={null}>
				<App />
			</Suspense>
		</I18nextProvider>
	</StrictMode>,
);
