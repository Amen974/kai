import { useRef, useState } from 'react';
import './App.css'

function App() {
  const message = useRef<HTMLInputElement>(null);
  const [response, setResponse] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const sendMessage = async () => {
    const currentMessage = message.current?.value;

    if (!currentMessage) return;

    setIsLoading(true);
    setError('');
    setResponse('');

    try {
      const result = await fetch('http://localhost:11434/api/generate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model: 'qwen3:4b',
          prompt: currentMessage,
          stream: false,
        }),
      });

      if (!result.ok) {
        throw new Error(`Ollama returned ${result.status}`);
      }

      const data: { response?: string } = await result.json();
      setResponse(data.response ?? 'No response received.');
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : 'Unable to reach Ollama.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <main className='grid min-h-screen place-items-center bg-[radial-gradient(circle_at_15%_10%,rgba(170,203,62,0.12),transparent_28%),linear-gradient(135deg,#111413_0%,#1a201e_52%,#101312_100%)] px-4.5 py-8 font-["Trebuchet_MS","Segoe_UI",sans-serif] text-[#f2f1eb]'>
      <section className='flex min-h-[min(720px,calc(100vh-64px))] w-full max-w-190 flex-col border border-[rgba(236,240,218,0.14)] bg-[rgba(24,29,27,0.88)] p-[clamp(24px,5vw,52px)] shadow-[0_28px_80px_rgba(0,0,0,0.3)] max-[480px]:min-h-screen max-[480px]:px-5 max-[480px]:py-7'>
        <header className='flex items-start justify-between border-b border-[rgba(236,240,218,0.12)] pb-7'>
          <div>
            <p className='mb-2.5 text-[0.68rem] uppercase tracking-[0.12em] text-[#b3d84c]'>LOCAL MODEL / READY</p>
            <h1 className='font-serif text-[clamp(2.2rem,6vw,4.3rem)] font-normal leading-[0.95] tracking-[-0.04em] text-[#f4f5ed]'>AI chat</h1>
          </div>
          <span className='mt-2 h-2.75 w-2.75 rounded-full bg-[#b3d84c] shadow-[0_0_0_5px_rgba(179,216,76,0.12)]' aria-label='Ollama connection status' />
        </header>

        <div className='flex flex-1 items-center py-10.5'>
          {!isLoading && !error && !response && (
            <div className='text-[#d5d9ce]'>
              <span className='mb-4 block text-[2.5rem] font-light leading-[0.7] text-[#b3d84c]'>+</span>
              <p className='mb-2 font-serif text-[1.55rem]'>Ask something to begin.</p>
              <small className='text-[0.68rem] tracking-[0.04em] text-[#9aa49b]'>Your prompt will be sent to llama3.2 locally.</small>
            </div>
          )}
          {isLoading && <p className='flex items-center gap-2.5 text-[#b3d84c]'><span className='h-1.75 w-1.75 animate-pulse rounded-full bg-[#b3d84c]' />Thinking...</p>}
          {error && <p className='m-0 text-[#f19b89]'>{error}</p>}
          {!isLoading && !error && response && <p className='m-0 w-full whitespace-pre-wrap border-l-[3px] border-[#b3d84c] px-6 py-5.5 text-[1.05rem] leading-7 text-[#f0f2e8]'>{response}</p>}
        </div>

        <div className='flex gap-2.5 border border-[rgba(236,240,218,0.18)] bg-[rgba(10,13,12,0.6)] p-1.75'>
          <input
            type='text'
            placeholder='Ask a question...'
            aria-label='Message'
            ref={message}
            className='min-w-0 flex-1 bg-transparent px-3.5 py-3 text-[#f4f5ed] outline-none placeholder:text-[#7d877e]'
            onKeyDown={(event) => {
              if (event.key === 'Enter') sendMessage();
            }}
          />
          <button type='button' className='cursor-pointer border-0 bg-[#b3d84c] px-4.25 text-[0.86rem] font-bold text-[#151912] hover:bg-[#c6ea5b] disabled:cursor-wait disabled:opacity-65 max-[480px]:px-3.25' onClick={sendMessage} disabled={isLoading}>
            {isLoading ? '...' : 'Send'} <span aria-hidden='true'>-&gt;</span>
          </button>
        </div>
        <p className='m-[15px_0_0] text-right text-[0.62rem] uppercase tracking-[0.08em] text-[#9aa49b]'>Powered by Ollama <span className='px-1.25 text-[#b3d84c]'>•</span> llama3.2</p>
      </section>
    </main>
  );
}

export default App;
