import styles from '@/features/games/ChessGame/components/styles/Cards.module.css';
import CardsModal from './CardsModal';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import Card from '@/features/games/ChessGame/components/Card';
import type { CardId, CardRegister } from '@/features/games/ChessGame/types/cardTypes';
import { CHESS_CARDS, cardImageUrl } from '@/features/games/ChessGame/types/cardTypes';
import { useLocation } from 'react-router-dom';
// import { p } from 'framer-motion/client';

interface CardsProps {
    hand: CardRegister[];
    isMyTurn: boolean;
    cardActionUsedThisTurn: boolean;
    onPlayCard: (cardId: CardId, target?: string) => void;
    onDiscardCard: (cardId: CardId) => void;
    onRequestTarget: (cardId: CardId) => void;
    pendingCardTarget: CardId | null;
}

export default function Cards({ hand, isMyTurn, cardActionUsedThisTurn, onPlayCard, onDiscardCard, onRequestTarget, pendingCardTarget }: CardsProps) {
    const { t } = useTranslation();
    const [activeIndex, setActiveIndex] = useState<number | null>(null);
    // const [devHand, setDevHand] = useState<CardRegister[]>([]);

    const location = useLocation();
    const isDevMode = location.pathname === "/dev/chess";

    // const hand = isDevMode && devHand.length > 0 ? devHand : hand;

    const currentIsMyTurn = isDevMode ? true : isMyTurn;

    const handleCardClick = (index: number) => {
        setActiveIndex(index);
    };

    const handlePlayCard = (cardId: CardId, target?: string) => {
        setActiveIndex(null);
        onPlayCard(cardId, target);
    };

    const handleDiscardCard = (cardId: CardId) => {
        setActiveIndex(null);
        onDiscardCard(cardId);
    };

    const handleRequestTarget = (cardId: CardId) => {
        setActiveIndex(null);
        onRequestTarget(cardId);
    };

    // const generatePlaceholderCards = () => {
    //     const availableCards = Object.keys(CHESS_CARDS) as CardId[];
    //     const mockCards = availableCards.slice(0, 5);
    //     setDevHand(mockCards);
    // };

    return (
        <>
            {hand.length === 0 && !isDevMode && (
                <p className={styles.emptyHand}>{t("games.noCards")}</p>
            )}

            {/* {isDevMode && hand.length === 0 && (
                <div style={{ textAlign: "center", margin: "1rem 0" }}>
                    <p style={{ color: "yellow", marginBottom: "0.5rem" }}>🛠️ Dev mode actif : Main vide</p>
                    <button
                        onClick={generatePlaceholderCards}
                        style={{
                            padding: "8px 16px",
                            backgroundColor: "yellow",
                            color: "black",
                            border: "none",
                            borderRadius: "4px",
                            cursor: "pointer",
                            fontWeight: "bold"
                        }}
                    >
                        Générer des cartes de test
                    </button>
                </div>
            )} */}

            <CardsModal
                hand={hand}
                activeIndex={activeIndex}
                isOpen={activeIndex !== null}
                close={() => setActiveIndex(null)}
                isMyTurn={currentIsMyTurn}
                cardActionUsedThisTurn={cardActionUsedThisTurn}
                onPlayCard={handlePlayCard}
                onDiscardCard={handleDiscardCard}
                onRequestTarget={handleRequestTarget}
                pendingCardTarget={pendingCardTarget}
            />

            <div className={styles.cards}>
                {hand.map((rawCard, index) => {
                    const card = CHESS_CARDS[rawCard.id];

                    if (!card) return null;

                    return (
                        <Card
                            zoom={false}
                            key={`${rawCard.id}-${index}`}
                            backgroundUrl={cardImageUrl(rawCard.id)}
                            title={t(`cards.${rawCard.id}.title`)}
                            handleCardClick={handleCardClick}
                            index={index}
                        />
                    );
                })}
            </div>
        </>
    );
}
