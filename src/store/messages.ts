import { create } from "zustand";
import { Message } from "../types";

type Messages = {
    messages: Message[]
    isLoading: boolean,
    currentId: number | null,
    setIsloading: (bool: boolean) => void,
    setMessages: (messages: Message[]) => void,
    addMessage: (newMessage: Message) => void
    addChunk: (newChunk: string) => void
    addThinkingChunk: (newChunk: string) => void
    setCurrentId: (id: number | null) => void,
}

const useMessages = create<Messages>((set) => ({
    messages: [],
    isLoading: false,
    currentId: null,

    setIsloading: (bool: boolean) => set({isLoading: bool}),

    setMessages: (messages: Message[]) => set({ messages }),

    addMessage: (newMessage: Message) => set((state) => ({
        messages: [...state.messages, newMessage]
    })),

    addChunk: (newChunk: string) => set((state) => ({
        messages: state.messages.map((message, index) =>
            index === state.messages.length - 1
                ? { ...message, content: message.content + newChunk }
                : message
        )
    })),

    addThinkingChunk: (newChunk: string) => set((state) => ({
        messages: state.messages.map((message, index) =>
            index === state.messages.length - 1
                ? { ...message, thinking: (message.thinking ?? "") + newChunk }
                : message
        )
    })),

    setCurrentId: (id: number | null) => set({ currentId: id }),
}))

export default useMessages;