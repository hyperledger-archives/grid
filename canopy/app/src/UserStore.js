import React from 'react';
import PropTypes from 'prop-types';

const UserContext = React.createContext();

export function UserProvider({ children }) {
  const userState = React.useState(null);
  return (
    <UserContext.Provider value={userState}>{children}</UserContext.Provider>
  );
}

export function useUserState() {
  const userState = React.useContext(UserContext);
  if (userState === undefined) {
    throw new Error('useUserState must be used within a UserProvider');
  }
  return userState;
}

UserProvider.propTypes = {
  children: PropTypes.node.isRequired
};
