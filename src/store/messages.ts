import { create } from "zustand";

type Messages = {
    messages: Message[]
    isLoading: boolean,
    setIsloading: (bool: boolean) => void,
    setMessages: (messages: Message[]) => void,
    addMessage: (newMessage: Message) => void
    addChunk: (newChunk: string) => void
}

const useMessages = create<Messages>((set) => ({
    messages: [],
    isLoading: false,

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
    }))
}))

export default useMessages;