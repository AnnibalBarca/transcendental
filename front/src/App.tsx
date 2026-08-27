import { BrowserRouter } from "react-router-dom";
import { AuthProvider } from "@/features/auth/context/AuthProvider";
import AppRoutes from "./AppRoutes";
import Footer from "./components/Footer";
import { Toaster } from "./components/ui/toast";

function App() {
	return (
		<>
			<BrowserRouter>
				<AuthProvider>
					<AppRoutes />
					<Footer />
				</AuthProvider>
			</BrowserRouter>
			<Toaster />
		</>
	);
}

export default App;
