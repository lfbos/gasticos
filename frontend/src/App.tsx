function App() {
  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Gastico
          </h1>
        </div>
      </header>
      <main>
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <div className="rounded-lg bg-white p-6 shadow">
            <h2 className="text-xl font-semibold text-gray-800">
              Analiza tus extractos bancarios
            </h2>
            <p className="mt-2 text-gray-600">
              Sube tus extractos de Bancolombia, Nequi, Nu Colombia o Davivienda
              y obtén insights sobre tus gastos.
            </p>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
