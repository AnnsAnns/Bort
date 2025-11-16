export enum Themes {
  latenightbath,
  ayy4,
  curiosities,
  sunnyswamp,
  standard_og,
  werwolvdark,
  nostalgia,
}

export const standard_theme = Themes.latenightbath;
export const theme_version = 3;
export const theme_cookie = "theme_v" + theme_version;

// TS Enums have no (good) way of finding the last element to prevent rollovers
// So it's either this or a cursed function
export const lastTheme = Themes.nostalgia;

export function loadSetTheme() {
  const theme = loadTheme();

  document.body.setAttribute("data-theme", Themes[theme]);
}

export function loadTheme(): number {
  let storedTheme: unknown = null;
  let retTheme = standard_theme;

  if (typeof window !== "undefined") {
    storedTheme = localStorage.getItem(theme_cookie);
  }

  if (typeof storedTheme === "number") {
    retTheme = storedTheme;
  } else if (typeof storedTheme === "string") {
    retTheme = parseInt(storedTheme);
  } else {
    retTheme = standard_theme;
  }

  return retTheme;
}

export function clickLoadTheme(e: Event) {
  e.preventDefault();
  const theme = loadTheme();
  const newTheme = theme < lastTheme ? theme + 1 : 0;
  localStorage.setItem(theme_cookie, newTheme.toString());

  // Reload the page to apply the new theme
  // This way we can have a static site with dynamic themes
  //location.reload();

  loadSetTheme();

  // Change text within "theme-name" div
  const themeName = document.querySelector(".theme-name");

  if (themeName) {
    themeName.textContent = Themes[newTheme];
  }
}
