import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { CARD_MODAL_POP_UP, TEXT_SOFT, BUTTON_PRIMARY, BUTTON_SECONDARY, BUTTON_ROW, CAROUSSEL_CARD, modalcardtitle } from "@/features/home/skin-room/components/skinRoomStyles";
import { CHESS_CARDS, cardImageDeckUrl, cardImageDeckVariant, maxRarityForCard, type CardId } from "@/data/cards";
import type { Card, DeckEntry } from "@/features/home/skin-room/context/CosmeticContext";
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
} from "@/components/ui/carousel"

type ModalParam = {
  selectedId: number | null;
  changeId: (id: number | null) => void;
  cards: Card[];
  deck: DeckEntry[];
  setDeckRarity: (cardId: string, rarity: number) => Promise<void>;
};

function rarityVariantUrl(id: CardId, rarity: number): string {
  return cardImageDeckVariant(id, rarity);
}

export default function CardModal({ selectedId, changeId, cards, deck, setDeckRarity }: ModalParam) {
  const { t } = useTranslation();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [side, setSide] = useState<number>(1);

  useEffect(() => {
    if (selectedId !== null && selectedId !== undefined) {
      dialogRef.current?.showModal();
    } else {
      dialogRef.current?.close();
    }
  }, [selectedId]);

  const idKey = String(selectedId) as CardId;
  const card = CHESS_CARDS[idKey] ?? null;
  const maxRarity = maxRarityForCard(idKey);
  const slides = Array.from({ length: maxRarity + 1 }, (_, i) => i);
  const owned = selectedId !== null && selectedId !== undefined
    ? cards.filter((c) => c.id === selectedId)
    : [];
  const ownedRarities = owned.map((c) => c.rarity);
  const selectedRarity = deck.find((d) => d.card_id === idKey)?.rarity ?? null;

  const handleSelect = async (rarity: number) => {
    try {
      await setDeckRarity(idKey, rarity);
    } catch (err) {
    }
  };

  return(
    <dialog
      ref={dialogRef}
      className={CARD_MODAL_POP_UP}
      onClose={() => changeId(null)}
      onClick={(e) => {
        if (e.target === dialogRef.current) changeId(null);
      }}
    >

      <div className={BUTTON_ROW}>
        <button
          className={side === 1 ? BUTTON_PRIMARY : BUTTON_SECONDARY}
          onClick={() => setSide(1)}
        >
          {t("skinroom.tabText")}
        </button>
        <button
          className={side === 2 ? BUTTON_PRIMARY : BUTTON_SECONDARY}
          onClick={() => setSide(2)}
        >
          {t("skinroom.tabSkins")}
        </button>
      </div>

      <div>
        {side === 1 && card && (
          <div className="flex flex-col items-center gap-3">
            <h2 className="m-0 text-[1.4rem] font-extrabold text-[#f5f5f0]">{t(`cards.${idKey}.title`)}</h2>
            <p className={TEXT_SOFT}>{t(`cards.${idKey}.description`)}</p>
          </div>
        )}
        {side === 1 && !card && (
          <p className={TEXT_SOFT}>{t("skinroom.unknownCard")}</p>
        )}
        {side === 2 && card && (
          <>
            <Carousel className={CAROUSSEL_CARD}>
              <CarouselContent>
                {slides.map((r) => (
                  <CarouselItem key={r}>
                    <div className="relative flex flex-col items-center gap-2">
                      <img
                        className={modalcardtitle(ownedRarities.includes(r))}
                        src={rarityVariantUrl(idKey, r)}
                        alt={`card_${selectedId}_rarity_${r}`}
                        loading="lazy"
                        decoding="async"
                        onError={(e) => {
                          e.currentTarget.src = cardImageDeckUrl(idKey);
                        }}
                      />
                      {ownedRarities.includes(r) ? (
                        <button
                          className={`w-fit cursor-pointer rounded-[2px] border border-[#334155]/60 px-3 py-1.5 text-sm font-semibold transition-colors duration-150 ${
                            selectedRarity === r
                              ? "bg-[#60a5fa] text-white"
                              : "bg-[#0f172a]/70 text-[#60a5fa] hover:bg-[#1e293b]"
                          }`}
                          onClick={() => handleSelect(r)}
                        >
                          {selectedRarity === r ? t("skinroom.equipped") : t("skinroom.equip")}
                        </button>
                      ) : (
                        <span className="text-xs text-white/40">{t("skinroom.notOwned")}</span>
                      )}
                    </div>
                  </CarouselItem>
                ))}
              </CarouselContent>
              <CarouselPrevious className="rounded-[2px] border-[#334155]/60 bg-[#0f172a]/80 text-white hover:bg-[#1e293b] hover:text-[#60a5fa]" />
              <CarouselNext className="rounded-[2px] border-[#334155]/60 bg-[#0f172a]/80 text-white hover:bg-[#1e293b] hover:text-[#60a5fa]" />
            </Carousel>
          </>
        )}
      </div>
    </dialog>
  )
}