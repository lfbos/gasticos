import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import esCO from './locales/es-CO.json';

const resources = {
  'es-CO': {
    translation: esCO,
  },
};

i18n.use(initReactI18next).init({
  resources,
  lng: 'es-CO', // Default to Colombian Spanish
  fallbackLng: 'es-CO',
  interpolation: {
    escapeValue: false, // React already escapes values
  },
});

export default i18n;
