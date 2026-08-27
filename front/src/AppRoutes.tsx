import { Navigate, Route, Routes } from "react-router-dom";
import { RouteGuard } from "@/features/auth/components/RouteGuard";
import { FriendProvider } from "@/features/friends/context/FriendContext";
import AuthPage from "@/pages/Auth/AuthPage";
import SetupPage from "@/pages/Auth/SetupPage";
import ProtectedHome from "./ProtectedHome";
import GamePage from "./pages/Games/GamePage";
import MatchmakingPage from "./pages/Games/MatchmakingPage";
import SpectateGame from "./pages/Games/SpectateGame";
//import LiveGamesPage from "./pages/Live/LiveGamesPage";
import CreateRoomPage from "./pages/Room/CreateRoomPage";
import RoomLobbyPage from "./pages/Room/RoomLobbyPage";
import ChessGameGuard from "./features/games/ChessGame/ChessGameGuard";
import ChessGame from "./features/games/ChessGame/ChessGame";
import ForgotPasswordPage from "./pages/Auth/ForgotPasswordPage";
import ResetPasswordPage from "./pages/Auth/ResetPasswordPage";
import ValidateEmailPage from "./pages/Auth/ValidateEmailPage";
import ChangeProviderPage from "./pages/Settings/ConnectEmailProviderPage";
import ChangePasswordPage from "./pages/Settings/ChangePasswordPage";
import PrivacyPolicyPage from "./pages/Legal/PrivacyPolicyPage";
import TermsOfServicePage from "./pages/Legal/TermsOfServicePage";
import PanelPage from "./pages/Admin/PanelPage";
import PlayerProfilePage from "./pages/Profile/PlayerProfilePage";

export default function AppRoutes() {
	return (
		<Routes>
			<Route path="/privacy" element={<PrivacyPolicyPage />} />
			<Route path="/terms" element={<TermsOfServicePage />} />
			<Route
				path="/login"
				element={
					<RouteGuard>
						<AuthPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/signup"
				element={
					<RouteGuard>
						<AuthPage />
					</RouteGuard>
				}
      />
      <Route
				path="/validate-email"
				element={
					<RouteGuard>
						<ValidateEmailPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/finish"
				element={
					<RouteGuard>
						<SetupPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/forgot-password"
				element={
					<RouteGuard>
						<ForgotPasswordPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/reset-password"
				element={
					<RouteGuard>
						<ResetPasswordPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/play"
				element={
					<RouteGuard>
						<ProtectedHome />
					</RouteGuard>
				}
			/>
			<Route
				path="/change-password"
				element={
					<RouteGuard>
						<ChangePasswordPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/connect-email-provider"
				element={
					<RouteGuard>
						<ChangeProviderPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/game"
				element={
					<RouteGuard>
						<GamePage />
					</RouteGuard>
				}
			/>
			<Route
				path="/game/matchmaking"
				element={
					<RouteGuard>
						<MatchmakingPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/game/chess"
				element={
					<RouteGuard>
						<ChessGameGuard />
					</RouteGuard>
				}
			/>
			<Route
				path="/game/spectate/:gameId"
				element={
					<RouteGuard>
						<SpectateGame />
					</RouteGuard>
				}
			/>
			<Route
				path="/profile/:id"
				element={
					<RouteGuard>
						<PlayerProfilePage />
					</RouteGuard>
				}
			/>
			{/* <Route
				path="/live"
				element={
					<RouteGuard>
						<LiveGamesPage />
					</RouteGuard>
				}
			/> */}
			<Route
				path="/room/create"
				element={
					<RouteGuard>
						<CreateRoomPage />
					</RouteGuard>
				}
			/>
			<Route
				path="/room/:id"
				element={
					<RouteGuard>
						<FriendProvider>
							<RoomLobbyPage />
						</FriendProvider>
					</RouteGuard>
				}
			/>
			<Route
				path="/panel"
				element={
					<RouteGuard>
						<PanelPage />
					</RouteGuard>
				}
			/>
			{/* <Route
				path="/dev/chess"
				element={
					<ChessGame />
				}
			/> */}
			<Route
				path="*"
				element={
					<RouteGuard>
						<Navigate to="/" replace />
					</RouteGuard>
				}
			/>
		</Routes>
	);
}
