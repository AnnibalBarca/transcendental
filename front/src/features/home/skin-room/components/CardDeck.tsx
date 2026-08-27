import { useContext, useState } from "react";
import { useTranslation } from "react-i18next";
import CardModal from "@/features/home/skin-room/components/CardModalText";
import { CARD_LIST, cardtitle, CARD_ITEM, CARD_DECK_TITLE } from "@/features/home/skin-room/components/skinRoomStyles";
import { SkinRoomContext } from "@/features/home/skin-room/context/CosmeticContext";
import { CHESS_CARDS, cardImageDeckUrl, cardImageDeckVariant, isCardId, type CardId } from "@/data/cards";

export default function CardDeck() {
  const { t } = useTranslation();
  const context = useContext(SkinRoomContext);
  const [selectedId, setSelectedId] = useState<number | null>(null);

  if (!context) return null;
  const { cards, deck, setDeckRarity } = context;

  const ids = Object.keys(CHESS_CARDS).map(Number);

  return (
    <div>
      <h3 className={CARD_DECK_TITLE}>{t("skinroom.deckTitle")}</h3>
      <ul className={CARD_LIST}>
        {ids.map((id) => {
          const owned = cards.some((element) => element.id === id);
          const idKey = String(id) as CardId;
          const deckEntry = deck.find((d) => d.card_id === idKey);
          const src = deckEntry
            ? cardImageDeckVariant(idKey, deckEntry.rarity)
            : isCardId(idKey)
              ? cardImageDeckUrl(idKey)
              : "";
          return (
            <li
              className={`relative ${cardtitle(owned)}`}
              key={id}
              onClick={() => setSelectedId(id)}
            >
              <img
                className={CARD_ITEM}
                src={src}
                alt={`card_${id}`}
                loading="lazy"
                decoding="async"
                onError={(e) => {
                  e.currentTarget.style.visibility = "hidden";
                }}
              />
              {owned && deckEntry && (
                <span className="pointer-events-none absolute right-1 bottom-1 rounded-[2px] bg-[#60a5fa] px-1.5 py-0.5 text-[10px] font-bold text-white">
                  {deckEntry.rarity}
                </span>
              )}
            </li>
          );
        })}
      </ul>

      <CardModal
        selectedId={selectedId}
        changeId={setSelectedId}
        cards={cards}
        deck={deck}
        setDeckRarity={setDeckRarity}
      />
    </div>
  );
}