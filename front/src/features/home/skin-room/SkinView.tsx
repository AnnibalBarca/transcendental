import CharacterEquipment from "@/features/home/skin-room/components/CharacterEquipment";
import { SkinRoomProvider } from "@/features/home/skin-room/context/CosmeticProvider";
import { SKIN_SCROLL } from "@/features/home/skin-room/components/skinRoomStyles";

export default function SkinView() {
  return (
      <SkinRoomProvider>
        <div className={SKIN_SCROLL}>
            <CharacterEquipment />
        </div>
      </SkinRoomProvider>
    );
}
