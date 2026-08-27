import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { toast } from "@/components/ui/toast";
import { SliderSection } from "@/features/play/components/SliderSection";
import { PlayView } from "@/features/play/components/PlayView";
import { PlayViewMatchmaking } from "@/features/play/components/PlayViewMatchmaking";
import { useSlider } from "@/features/play/contexts/SliderContext";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { roomService } from "@/features/room/services/roomService";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { RoomWaitingPanel } from "@/features/play/components/RoomWaitingPanel";
import { useState, useCallback, useMemo } from "react"; // Ajout des hooks
import type { GameMode } from "@/features/play/components/ModeSelector";
import PublicRooms from "@/features/room/components/PublicRooms";

const TIME_CONTROLS = ["5", "10", "15"];

export default function HomePage() {
    const { t } = useTranslation();
    const { setActiveIndex, setIsSwipeDisabled } = useSlider();
    const navigate = useNavigate();
    const { user, userState, chessGameId, roomId } = useAuth();
    const [gameMode, setGameMode] = useState<GameMode>(
        () => (sessionStorage.getItem("game_mode") as GameMode) ?? "ranked",
    );

    const isInGame = userState === "playing" && !!chessGameId;
    const isMatchmaking = userState === "matchmaking";
    const isInRoom = userState === "waiting" && !!roomId;

    const handleJoinClick = useCallback(() => {
        navigate("/game/matchmaking");
    }, [navigate]);

    const handleModeChange = useCallback((mode: GameMode, option: string) => {
        setGameMode(mode);
        sessionStorage.setItem("game_mode", mode);
        const minutes = parseInt(option, 10);
        if (TIME_CONTROLS.includes(String(minutes))) {
            sessionStorage.setItem("chess_time_control", String(minutes));
        }
    }, []);

    const handleRejoin = useCallback(() => {
        navigate("/game/chess");
    }, [navigate]);

    const handleAbandon = useCallback(async () => {
        if (!window.confirm(t("home.confirmAbandon"))) return;
        try {
            await roomService.leaveGame();
            window.location.reload();
            toast.add(
                {
                    title: t("home.abandoned"),
                    type: "success",
                }
            )
        } catch (e) {
            toast.add(
                {
                    title: t("home.abandonFailed"),
                    type: "error",
                }
            )
        }
    }, [t]);

    const handleOfferClick = useCallback(() => setActiveIndex(0), [setActiveIndex]);
    const handleCreateClick = useCallback(() => navigate("/room/create"), [navigate]);
    //const handleLiveClick = useCallback(() => navigate("/live"), [navigate]);

const actionAreaJSX = useMemo(() => (
        <div className="flex items-center justify-center gap-4">
            <ThemeButton
                type="button"
                texturePosition="center 95%"
                textureZoom={100}
                onClick={handleRejoin}
                className="h-14 min-w-0 flex-1 px-4"
            >
                <span className="text-sm tracking-[2px] uppercase">{t("home.rejoin")}</span>
            </ThemeButton>
            <ThemeButton
                type="button"
                tone="red"
                texturePosition="center 95%"
                textureZoom={100}
                onClick={handleAbandon}
                className="h-14 min-w-0 flex-1 px-4"
            >
                <span className="text-sm tracking-[2px] uppercase">{t("home.abandon")}</span>
            </ThemeButton>
        </div>
    ), [handleRejoin, handleAbandon, t]);

    const footerJSX = useMemo(() => <PublicRooms />, []);

    return (
        <SliderSection>
            {isInGame ? (
                <PlayView
                    onOfferClick={handleOfferClick}
                    onSwipeToggle={setIsSwipeDisabled}
                    onJoinClick={handleJoinClick}
                    hideModeSelector
                    actionArea={actionAreaJSX}
                />
            ) : isInRoom ? (
                <PlayView
                    onOfferClick={handleOfferClick}
                    onSwipeToggle={setIsSwipeDisabled}
                    onJoinClick={handleJoinClick}
                    hideModeSelector
                    actionArea={<RoomWaitingPanel />}
                />
            ) : isMatchmaking ? (
                <PlayViewMatchmaking
                    onOfferClick={handleOfferClick}
                    onJoinClick={handleJoinClick}
                />
            ) : (
                <PlayView
                    onOfferClick={handleOfferClick}
                    onSwipeToggle={setIsSwipeDisabled}
                    onJoinClick={handleJoinClick}
                    onCreateClick={handleCreateClick}
                    //onLiveClick={handleLiveClick}
                    onModeChange={handleModeChange}
                    joinLabel={t("home.play")}
                    playerElo={user?.ranked_elo ?? 1500}
                    footer={footerJSX}
                />
            )}
        </SliderSection>
    );
}
