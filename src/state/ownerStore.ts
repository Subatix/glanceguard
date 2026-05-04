import { create } from "zustand";

type OwnerState = {
  ownerEnrolled: boolean;
  setOwnerEnrolled: (value: boolean) => void;
};

export const useOwnerStore = create<OwnerState>()((set) => ({
  ownerEnrolled: false,
  setOwnerEnrolled: (ownerEnrolled) => set({ ownerEnrolled }),
}));
